use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::state::AppState;

/// Render the contacts list view
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_contact_list(f, chunks[0], state);
    render_contact_detail(f, chunks[1], state);
}

/// Render the contact list sidebar
fn render_contact_list(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title("👤 Contacts")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let contacts = &state.entities.contacts;

    if contacts.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No contacts yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'n' to add a contact",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    // Sort by online status (online first), then by unread count, then by name
    let mut sorted_contacts = contacts.clone();
    sorted_contacts.sort_by(|a, b| {
        // Online contacts first (recent last_seen)
        let a_online = is_online(a.last_seen);
        let b_online = is_online(b.last_seen);

        b_online
            .cmp(&a_online)
            .then_with(|| b.unread_count.cmp(&a.unread_count))
            .then_with(|| a.display_name.cmp(&b.display_name))
    });

    let items: Vec<ListItem> = sorted_contacts
        .iter()
        .enumerate()
        .map(|(i, contact)| {
            let is_selected = i == state.navigation.selected_index;
            let prefix = if is_selected { "→ " } else { "  " };

            // Online/offline indicator
            let status_indicator = if is_online(contact.last_seen) {
                "🟢"
            } else {
                "⚪"
            };

            let unread_badge = if contact.unread_count > 0 {
                format!(" ({})", contact.unread_count)
            } else {
                String::new()
            };

            let last_seen_str = format_last_seen(contact.last_seen);

            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if contact.unread_count > 0 {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_online(contact.last_seen) {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };

            let text = format!(
                "{}{} {}{}\n    {} • {}",
                prefix,
                status_indicator,
                contact.display_name,
                unread_badge,
                contact.four_words,
                last_seen_str
            );
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

/// Render the contact detail/preview panel
fn render_contact_detail(f: &mut Frame, area: Rect, state: &AppState) {
    let contacts = &state.entities.contacts;

    if contacts.is_empty() {
        let block = Block::default()
            .title("Contact Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(block, area);
        return;
    }

    let mut sorted_contacts = contacts.clone();
    sorted_contacts.sort_by(|a, b| {
        let a_online = is_online(a.last_seen);
        let b_online = is_online(b.last_seen);

        b_online
            .cmp(&a_online)
            .then_with(|| b.unread_count.cmp(&a.unread_count))
            .then_with(|| a.display_name.cmp(&b.display_name))
    });

    let selected_contact = match sorted_contacts.get(state.navigation.selected_index) {
        Some(contact) => contact,
        None => {
            let block = Block::default()
                .title("Contact Details")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(block, area);
            return;
        }
    };

    let online = is_online(selected_contact.last_seen);
    let status_color = if online { Color::Green } else { Color::Gray };
    let status_text = if online { "Online" } else { "Offline" };

    let block = Block::default()
        .title(format!("👤 {}", selected_contact.display_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Identity: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                &selected_contact.four_words,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    // Last seen
    if let Some(last_seen) = selected_contact.last_seen {
        lines.push(Line::from(vec![
            Span::styled("Last seen: ", Style::default().fg(Color::Yellow)),
            Span::raw(format_timestamp(last_seen)),
        ]));
        lines.push(Line::from(""));
    }

    // Unread messages
    if selected_contact.unread_count > 0 {
        lines.push(Line::from(vec![
            Span::styled("Unread: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} messages", selected_contact.unread_count),
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
    lines.push(Line::from(Span::styled(
        "Press Enter to open direct messages",
        Style::default().fg(Color::Green),
    )));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Check if a contact is online (last seen within 5 minutes)
fn is_online(last_seen: Option<i64>) -> bool {
    if let Some(timestamp) = last_seen {
        use std::time::{SystemTime, UNIX_EPOCH};

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let now = duration.as_secs() as i64;
        let diff = now - timestamp;

        diff < 300 // 5 minutes
    } else {
        false
    }
}

/// Format last seen timestamp
fn format_last_seen(last_seen: Option<i64>) -> String {
    if let Some(timestamp) = last_seen {
        if is_online(Some(timestamp)) {
            "Active now".to_string()
        } else {
            format!("Last seen {}", format_timestamp(timestamp))
        }
    } else {
        "Never seen".to_string()
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
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}

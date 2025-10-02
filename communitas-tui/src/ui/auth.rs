use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::{navigation::FocusedPanel, AppState};

/// Render the authentication view (login or signup)
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Header
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    render_header(f, chunks[0]);
    render_auth_options(f, chunks[1], state);
    render_instructions(f, chunks[2], state);
}

/// Render the welcome header
fn render_header(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let header_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "🌐 Welcome to Communitas",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            "A decentralized collaboration platform",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
    ];

    let paragraph = Paragraph::new(header_text)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Render authentication options (login or signup)
fn render_auth_options(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_login_option(f, chunks[0], state);
    render_signup_option(f, chunks[1], state);
}

/// Render login option card
fn render_login_option(f: &mut Frame, area: Rect, state: &AppState) {
    let is_selected = state.navigation.selected_index == 0;

    let border_color = if is_selected {
        Color::Green
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title("🔑 Login")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Login with existing identity",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    if is_selected && matches!(state.navigation.focused_panel, FocusedPanel::Input) {
        lines.push(Line::from(Span::styled(
            "Enter four-word identity:",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        // Show input field
        let input_text = state.input_buffer.as_str();
        let input_line = if input_text.is_empty() {
            Line::from(Span::styled(
                "ocean-forest-moon-star",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Line::from(Span::styled(
                input_text,
                Style::default().fg(Color::White),
            ))
        };
        lines.push(input_line);
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press Enter to login",
            Style::default().fg(Color::Green),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Already have an identity?",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            if is_selected {
                "→ Press Enter to login"
            } else {
                "  Use arrow keys to select"
            },
            Style::default().fg(if is_selected { Color::Green } else { Color::DarkGray }),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Render signup option card
fn render_signup_option(f: &mut Frame, area: Rect, state: &AppState) {
    let is_selected = state.navigation.selected_index == 1;

    let border_color = if is_selected {
        Color::Green
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title("✨ Signup")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Create new identity",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    if is_selected && matches!(state.navigation.focused_panel, FocusedPanel::Input) {
        lines.push(Line::from(Span::styled(
            "Enter display name:",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        // Show input field
        let input_text = state.input_buffer.as_str();
        let input_line = if input_text.is_empty() {
            Line::from(Span::styled(
                "Your Name",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Line::from(Span::styled(
                input_text,
                Style::default().fg(Color::White),
            ))
        };
        lines.push(input_line);
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press Enter to create identity",
            Style::default().fg(Color::Green),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Generate a new four-word identity",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            if is_selected {
                "→ Press Enter to signup"
            } else {
                "  Use arrow keys to select"
            },
            Style::default().fg(if is_selected { Color::Green } else { Color::DarkGray }),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Render instructions at the bottom
fn render_instructions(f: &mut Frame, area: Rect, state: &AppState) {
    let instructions = if matches!(state.navigation.focused_panel, FocusedPanel::Input) {
        "ESC: Cancel | Enter: Submit | Type to enter text"
    } else {
        "←/→: Switch option | Enter: Select | q: Quit"
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(Line::from(Span::styled(
        instructions,
        Style::default().fg(Color::Yellow),
    )))
    .block(block)
    .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

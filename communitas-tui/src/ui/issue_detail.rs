use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::state::AppState;

/// Render issue detail view
pub fn render(f: &mut Frame, area: Rect, state: &AppState, issue_id: &str) {
    // Find the issue across all projects
    let issue = find_issue_by_id(state, issue_id);

    if let Some(issue) = issue {
        render_issue_detail(f, area, state, issue);
    } else {
        render_issue_not_found(f, area, issue_id);
    }
}

/// Find issue by ID across all projects
fn find_issue_by_id<'a>(
    state: &'a AppState,
    issue_id: &str,
) -> Option<&'a crate::state::entities::IssueData> {
    for issues_list in state.entities.issues.values() {
        if let Some(issue) = issues_list.iter().find(|i| i.id == issue_id) {
            return Some(issue);
        }
    }
    None
}

/// Render issue detail
fn render_issue_detail(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    issue: &crate::state::entities::IssueData,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Issue header (title, status, priority)
            Constraint::Length(7), // Issue metadata (assignee, reporter, etc.)
            Constraint::Min(5),    // Description and comments
        ])
        .split(area);

    render_issue_header(f, chunks[0], issue);
    render_issue_metadata(f, chunks[1], state, issue);
    render_issue_description(f, chunks[2], issue);
}

/// Render issue header with title and status
fn render_issue_header(f: &mut Frame, area: Rect, issue: &crate::state::entities::IssueData) {
    let block = Block::default()
        .title(" Issue Detail ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let status_color = match issue.status.to_lowercase().as_str() {
        "backlog" => Color::Gray,
        "todo" => Color::Blue,
        "in_progress" => Color::Yellow,
        "done" => Color::Green,
        "canceled" => Color::Red,
        _ => Color::White,
    };

    let priority_color = match issue.priority.to_lowercase().as_str() {
        "critical" | "urgent" => Color::Red,
        "high" => Color::Yellow,
        "medium" => Color::Blue,
        "low" => Color::Gray,
        _ => Color::White,
    };

    let header_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            &issue.title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &issue.status,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  │  "),
            Span::styled("Priority: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &issue.priority,
                Style::default()
                    .fg(priority_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(header_lines)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render issue metadata
fn render_issue_metadata(
    f: &mut Frame,
    area: Rect,
    _state: &AppState,
    issue: &crate::state::entities::IssueData,
) {
    let block = Block::default()
        .title(" Metadata ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let assignee_text = if let Some(assignee_id) = &issue.assignee_id {
        assignee_id.as_str()
    } else {
        "Unassigned"
    };

    let metadata_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Issue ID: ", Style::default().fg(Color::Gray)),
            Span::styled(&issue.id, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Assignee: ", Style::default().fg(Color::Gray)),
            Span::styled(assignee_text, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Reporter: ", Style::default().fg(Color::Gray)),
            Span::styled(&issue.reporter_id, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Comments: ", Style::default().fg(Color::Gray)),
            Span::styled(
                issue.comment_count.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(metadata_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render issue description
fn render_issue_description(f: &mut Frame, area: Rect, issue: &crate::state::entities::IssueData) {
    let block = Block::default()
        .title(" Description ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if let Some(description) = &issue.description {
        if description.is_empty() {
            let empty_lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No description provided",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )),
            ];
            let paragraph = Paragraph::new(empty_lines)
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(paragraph, area);
        } else {
            let description_lines: Vec<Line> = description
                .lines()
                .map(|line| Line::from(Span::raw(line)))
                .collect();

            let paragraph = Paragraph::new(description_lines)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);
        }
    } else {
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No description provided",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'e' to edit this issue",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(empty_lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

/// Render issue not found message
fn render_issue_not_found(f: &mut Frame, area: Rect, issue_id: &str) {
    let block = Block::default()
        .title(" Issue Not Found ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let error_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Issue not found",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::styled(issue_id, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' to go back",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(error_lines)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::state::AppState;

/// Render the projects list view
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_project_list(f, chunks[0], state);
    render_project_detail(f, chunks[1], state);
}

/// Render the project list sidebar
fn render_project_list(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title("📋 Projects")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if state.entities.projects.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No projects yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Create a project to get started",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = state
        .entities
        .projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let is_selected = i == state.navigation.selected_index;
            let prefix = if is_selected { "→ " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let icon = project.icon.as_deref().unwrap_or("📋");
            let text = format!("{}{} {}", prefix, icon, project.name);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

/// Render the project detail/preview panel
fn render_project_detail(f: &mut Frame, area: Rect, state: &AppState) {
    if state.entities.projects.is_empty() {
        let block = Block::default()
            .title("Project Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(block, area);
        return;
    }

    let selected_project = match state.entities.projects.get(state.navigation.selected_index) {
        Some(project) => project,
        None => {
            let block = Block::default()
                .title("Project Details")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(block, area);
            return;
        }
    };

    let block = Block::default()
        .title(format!("{} {}", selected_project.icon.as_deref().unwrap_or("📋"), selected_project.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Yellow)),
            Span::raw(
                selected_project
                    .description
                    .clone()
                    .unwrap_or_else(|| "No description".to_string()),
            ),
        ]),
        Line::from(""),
    ];

    // Show issue counts by status
    let issue_counts = count_issues_by_status(state, &selected_project.id);
    if !issue_counts.is_empty() {
        lines.push(Line::from(Span::styled(
            "Issue Status:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (status, count) in issue_counts {
            let (status_str, color) = match status.as_str() {
                "backlog" => ("📦 Backlog", Color::Gray),
                "todo" => ("📝 Todo", Color::Blue),
                "in-progress" => ("🚀 In Progress", Color::Yellow),
                "done" => ("✅ Done", Color::Green),
                "canceled" => ("❌ Canceled", Color::DarkGray),
                _ => ("❓ Unknown", Color::White),
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{}: ", status_str), Style::default().fg(color)),
                Span::raw(format!("{} issues", count)),
            ]));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Enter to view Kanban board",
        Style::default().fg(Color::Green),
    )));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Count issues by status for a project
fn count_issues_by_status(state: &AppState, project_id: &str) -> Vec<(String, usize)> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Get issues for this project
    if let Some(issues) = state.entities.issues.get(project_id) {
        for issue in issues {
            *counts.entry(issue.status.clone()).or_insert(0) += 1;
        }
    }

    let mut result: Vec<(String, usize)> = counts.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Render Kanban board view for a project
pub fn render_kanban_board(f: &mut Frame, area: Rect, state: &AppState, project_id: &str) {
    // Find the project
    let project = state.entities.projects.iter().find(|p| p.id == project_id);

    let title = if let Some(proj) = project {
        format!("🎯 {} - Kanban Board", proj.name)
    } else {
        "Kanban Board".to_string()
    };

    // Split into 5 columns: backlog, todo, in-progress, done, canceled
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Backlog
            Constraint::Percentage(20), // Todo
            Constraint::Percentage(20), // In Progress
            Constraint::Percentage(20), // Done
            Constraint::Percentage(20), // Canceled
        ])
        .split(area);

    render_kanban_column(f, chunks[0], state, project_id, "backlog", "📦 Backlog", Color::Gray);
    render_kanban_column(f, chunks[1], state, project_id, "todo", "📝 Todo", Color::Blue);
    render_kanban_column(f, chunks[2], state, project_id, "in-progress", "🚀 In Progress", Color::Yellow);
    render_kanban_column(f, chunks[3], state, project_id, "done", "✅ Done", Color::Green);
    render_kanban_column(f, chunks[4], state, project_id, "canceled", "❌ Canceled", Color::DarkGray);
}

/// Render a single Kanban column
fn render_kanban_column(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    project_id: &str,
    status: &str,
    title: &str,
    color: Color,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    // Get issues for this project and filter by status
    let issues: Vec<_> = state
        .entities
        .issues
        .get(project_id)
        .map(|project_issues| {
            project_issues
                .iter()
                .filter(|issue| issue.status == status)
                .collect()
        })
        .unwrap_or_default();

    if issues.is_empty() {
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No issues",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = issues
        .iter()
        .map(|issue| {
            let priority_icon = match issue.priority.as_str() {
                "urgent" => "🔴",
                "high" => "🟠",
                "medium" => "🟡",
                "low" => "🔵",
                _ => "⚪",
            };

            let text = format!("{} {}", priority_icon, issue.title);
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

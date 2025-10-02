use crate::state::{AppState, View};
use crate::ui::{auth, dashboard, organizations, projects, status_bar};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Render main layout with status bar at bottom
pub fn render_layout(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),      // Main content
            Constraint::Length(1),    // Status bar
        ])
        .split(f.area());

    // Render main content based on current view
    render_main_content(f, chunks[0], state);

    // Render status bar
    status_bar::render(f, chunks[1], state);
}

/// Render main content area based on current view
fn render_main_content(f: &mut Frame, area: Rect, state: &AppState) {
    match state.navigation.current_view() {
        View::Auth => auth::render(f, area, state),
        View::Dashboard => dashboard::render(f, area, state),
        View::Organizations => render_organizations(f, area, state),
        View::Channel { .. } => render_channel(f, area, state),
        View::Thread { .. } => render_thread(f, area, state),
        View::Projects => render_projects(f, area, state),
        View::ProjectIssues { .. } => render_project_issues(f, area, state),
        View::IssueDetail { .. } => render_issue_detail(f, area, state),
        View::Groups => render_groups(f, area, state),
        View::GroupMessages { .. } => render_group_messages(f, area, state),
        View::Contacts => render_contacts(f, area, state),
        View::DirectMessages { .. } => render_direct_messages(f, area, state),
        View::NetworkStatus => render_network_status(f, area, state),
        View::Help => render_help(f, area, state),
    }
}

// Stub implementations for each view
fn render_organizations(f: &mut Frame, area: Rect, state: &AppState) {
    organizations::render(f, area, state);
}

fn render_channel(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::Channel { channel_id } = state.navigation.current_view() {
        organizations::render_channel_view(f, area, state, channel_id);
    }
}

fn render_thread(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::Thread { channel_id, thread_id } = state.navigation.current_view() {
        organizations::render_thread_view(f, area, state, channel_id, thread_id);
    }
}

fn render_projects(f: &mut Frame, area: Rect, state: &AppState) {
    projects::render(f, area, state);
}

fn render_project_issues(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::ProjectIssues { project_id } = state.navigation.current_view() {
        projects::render_kanban_board(f, area, state, project_id);
    }
}

fn render_issue_detail(_f: &mut Frame, _area: Rect, _state: &AppState) {
    // TODO: Implement issue detail view
}

fn render_groups(_f: &mut Frame, _area: Rect, _state: &AppState) {
    // TODO: Implement groups view
}

fn render_group_messages(_f: &mut Frame, _area: Rect, _state: &AppState) {
    // TODO: Implement group messages view
}

fn render_contacts(_f: &mut Frame, _area: Rect, _state: &AppState) {
    // TODO: Implement contacts view
}

fn render_direct_messages(_f: &mut Frame, _area: Rect, _state: &AppState) {
    // TODO: Implement direct messages view
}

fn render_network_status(_f: &mut Frame, _area: Rect, _state: &AppState) {
    // TODO: Implement network status view
}

fn render_help(_f: &mut Frame, _area: Rect, _state: &AppState) {
    // TODO: Implement help view
}
